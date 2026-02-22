use jeebs::plugins::load_dynamic_plugins;

#[test]
#[ignore] // Requires plugin runtimes (python3/node) to be installed
fn discover_examples() {
    let plugins = load_dynamic_plugins("plugins");
    // at least one of the sample plugins should be discovered
    let names: Vec<&str> = plugins.iter().map(|p| p.name()).collect();
    assert!(
        names.contains(&"python-echo") || names.contains(&"node-hello"),
        "expected example plugins to be loadable"
    );
}
