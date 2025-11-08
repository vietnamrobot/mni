// Test various cases
function test1(x) {
  const y = x || "default";
  y.value = 42;
}

function test2(a) {
  if (!a) return;
  const b = a.find() || a;
  b.text = "test";
}

const obj = {
  prop: getValue() || fallback
};
obj.prop.method();

function updateSettings(config) {
  const settings = config.settings || getDefaults();
  settings.enabled = config.enabled !== false;
  return settings;
}
