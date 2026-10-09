const { test } = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const ts = require('typescript');

function loadTs(file, mocks = {}) {
  const source = fs.readFileSync(path.join(__dirname, '..', file), 'utf8');
  const code = ts.transpileModule(source, {
    compilerOptions: { module: ts.ModuleKind.CommonJS, target: ts.ScriptTarget.ES2020 }
  }).outputText;
  const module = { exports: {} };
  new Function('require', 'module', 'exports', code)(name => {
    assert.ok(name in mocks, `Unexpected import: ${name}`);
    return mocks[name];
  }, module, module.exports);
  return module.exports;
}

const planner = loadTs('src/utils/reminderPlan.ts');
const periods = [
  { start: '08:00', end: '08:45' }, { start: '08:50', end: '09:35' },
  { start: '09:50', end: '10:35' }, { start: '10:40', end: '11:25' }
];
const firstDay = new Date(2030, 0, 7).getTime() / 1000; // Monday, local time
const course = (name, range, weeks = [1, 3]) => ({
  name, teacher: '', location: '主教101', periods: range, weeks, weekType: 1, dayOfWeek: 1
});
const now = new Date(2030, 0, 1);

test('actual dates, exact weeks, adjacency and full course time', () => {
  const plan = planner.buildReminderPlan([
    course('前课', [1, 2]), course('后课', [3, 4], [1, 2, 3])
  ], periods, firstDay, 2, now);
  assert.equal(plan.length, 3);
  assert.equal(plan[0].at.getHours(), 7);
  assert.equal(plan[0].at.getMinutes(), 45);
  assert.match(plan[0].body, /08:00 - 09:35/);
  assert.equal(plan[1].course.name, '后课');
  assert.equal(plan[1].at.getHours(), 9);
  assert.equal(plan[1].at.getMinutes(), 32);
  assert.match(plan[1].body, /09:50 - 11:25/);
  assert.equal(plan[2].at.getDate(), 14);
  assert.equal(plan[2].at.getMinutes(), 35); // no previous course in week 2
  assert.equal(new Set(plan.map(item => item.id)).size, plan.length);
});

test('exams do not suppress class reminders; expired/invalid entries are excluded', () => {
  const courses = [
    { ...course('考试', [1, 2]), courseType: 'exam' },
    course('课程', [3, 4]), course('坏数据', [99, 100])
  ];
  const plan = planner.buildReminderPlan(courses, periods, firstDay, 20, now);
  assert.equal(plan.length, 2);
  assert.equal(plan[0].at.getMinutes(), 35);
  assert.deepEqual(planner.buildReminderPlan(courses, periods, undefined, 20, now), []);
  assert.deepEqual(planner.buildReminderPlan(courses, periods, firstDay, 20, new Date(2031, 0, 1)), []);
  assert.deepEqual(planner.buildReminderPlan([course('课程', [1, 1])], [
    { start: '25:00', end: '26:00' }
  ], firstDay, 20, now), []);
});

function serviceHarness() {
  Object.defineProperty(globalThis, 'navigator', { value: { userAgent: 'Android' }, configurable: true });
  const pendingItems = [];
  const notifications = [];
  const errors = [];
  let unmount;
  let permission = async () => true;
  let delivery = async options => {
    notifications.push(options);
    if (options.schedule) pendingItems.push(options);
  };
  const { useReminder } = loadTs('src/composables/useReminder.ts', {
    vue: { ref: value => ({ value }), onUnmounted: callback => { unmount = callback; } },
    '@tauri-apps/api/core': { invoke: async (command, args) => {
      if (command.endsWith('is_permission_granted')) return permission();
      if (command.endsWith('request_permission')) return 'granted';
      if (command.endsWith('notify')) return delivery(args.options);
      assert.fail(`Unexpected command: ${command}`);
    } },
    '@tauri-apps/plugin-notification': {
      pending: async () => [...pendingItems],
      cancel: async ids => {
        for (let i = pendingItems.length - 1; i >= 0; i--) {
          if (ids.includes(pendingItems[i].id)) pendingItems.splice(i, 1);
        }
      },
      createChannel: async () => {}, Importance: { High: 4 },
      Schedule: { at: (date, repeating, allowWhileIdle) => ({ at: { date, repeating, allowWhileIdle } }) }
    },
    'element-plus': { ElMessage: { warning: () => {}, success: () => {}, error: text => errors.push(text) } },
    '../utils/reminderPlan': planner
  });
  return {
    service: useReminder(), pendingItems, notifications, errors, unmount: () => unmount(),
    setPermission: fn => { permission = fn; }, setDelivery: fn => { delivery = fn; }
  };
}
const tables = [{ id: 'test', name: 'test', periods }];

test('native schedules survive unmount; switching and disabling cancel only course alarms', async () => {
  const h = serviceHarness();
  h.pendingItems.push({ id: 42 });
  await h.service.startReminderService([course('原课', [1, 2], [1])], tables, firstDay, 20);
  assert.equal(h.notifications.length, 1);
  assert.equal(h.notifications[0].schedule.at.allowWhileIdle, true);
  await h.service.startReminderService([course('新课', [3, 4], [1])], tables, firstDay, 20);
  assert.equal(h.pendingItems.length, 2);
  assert.match(h.pendingItems[1].body, /新课/);
  h.unmount();
  assert.equal(h.pendingItems.length, 2);
  await h.service.stopReminderService();
  assert.deepEqual(h.pendingItems, [{ id: 42 }]);
});

test('disabling during a permission request cannot schedule obsolete reminders', async () => {
  const h = serviceHarness();
  let release;
  let requested;
  const entered = new Promise(resolve => { requested = resolve; });
  h.setPermission(() => {
    requested();
    return new Promise(resolve => { release = resolve; });
  });
  const start = h.service.startReminderService([course('课程', [1, 2])], tables, firstDay, 20);
  await entered;
  const stop = h.service.stopReminderService();
  release(true);
  await Promise.all([start, stop]);
  assert.equal(h.notifications.length, 0);
  assert.equal(h.service.isRunning.value, false);
});

test('native delivery failure is surfaced instead of reporting test success', async () => {
  const h = serviceHarness();
  h.setDelivery(async () => { throw new Error('native failure'); });
  await h.service.testNotification();
  assert.equal(h.errors.length, 1);
  assert.match(h.errors[0], /native failure/);
});
