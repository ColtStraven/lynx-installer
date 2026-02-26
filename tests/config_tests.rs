/// tests/config_tests.rs
///
/// Integration tests for .lynx project file parsing.

use lynx_engine::config::{InstallStep, LynxProject, Prerequisite, ShortcutLocation};

fn sample_project_toml() -> String {
    let lines = vec![
        "[app]",
        "name = \"My Awesome App\"",
        "version = \"2.1.0\"",
        "publisher = \"Acme Corp\"",
        "id = \"com.acme.myawesomeapp\"",
        "icon = \"assets/icon.ico\"",
        "default_install_dir = \"{program_files}/MyAwesomeApp\"",
        "url = \"https://acme.com\"",
        "description = \"The most awesome app ever made.\"",
        "",
        "[theme]",
        "name = \"lynx-default\"",
        "path = \"themes/lynx-default\"",
        "accent_color = \"#FF6B35\"",
        "background_color = \"#1A1A2E\"",
        "",
        "[theme.custom_vars]",
        "hero_text = \"Welcome to My Awesome App\"",
        "logo_path = \"assets/logo.png\"",
        "",
        "[[files]]",
        "source = \"dist/\"",
        "destination = \"{install_dir}\"",
        "recursive = true",
        "",
        "[[files]]",
        "source = \"assets/icon.ico\"",
        "destination = \"{install_dir}/icon.ico\"",
        "recursive = false",
        "",
        "[[steps]]",
        "kind = \"extract\"",
        "label = \"Installing application files...\"",
        "",
        "[[steps]]",
        "kind = \"shortcut\"",
        "label = \"Creating desktop shortcut...\"",
        "target = \"{install_dir}/myapp.exe\"",
        "locations = [\"desktop\", \"start_menu\"]",
        "name = \"My Awesome App\"",
        "",
        "[[steps]]",
        "kind = \"registry\"",
        "label = \"Writing registry entries...\"",
        "hive = \"HKEY_LOCAL_MACHINE\"",
        "key = \"SOFTWARE\\\\AcmeCorp\\\\MyAwesomeApp\"",
        "value_name = \"InstallPath\"",
        "value_data = \"{install_dir}\"",
        "value_type = \"REG_SZ\"",
        "",
        "[[steps]]",
        "kind = \"register_uninstaller\"",
        "label = \"Registering uninstaller...\"",
        "",
        "[[prerequisites]]",
        "kind = \"disk_space\"",
        "required_mb = 250",
        "",
        "[[prerequisites]]",
        "kind = \"dot_net\"",
        "minimum_version = \"6.0\"",
        "download_url = \"https://dotnet.microsoft.com/download\"",
        "",
        "[uninstall]",
        "enabled = true",
        "exe_name = \"uninstall.exe\"",
        "remove_install_dir = true",
        "extra_remove_paths = [\"{app_data}/AcmeCorp/MyAwesomeApp\"]",
    ];
    lines.join("\n")
}

#[test]
fn test_parse_full_project() {
    let toml = sample_project_toml();
    let project = LynxProject::from_toml_str(&toml).expect("Should parse successfully");

    // App metadata
    assert_eq!(project.app.name, "My Awesome App");
    assert_eq!(project.app.version, "2.1.0");
    assert_eq!(project.app.publisher, "Acme Corp");
    assert_eq!(project.app.id, "com.acme.myawesomeapp");
    assert_eq!(project.app.icon, Some("assets/icon.ico".to_string()));

    // Theme
    assert_eq!(project.theme.name, "lynx-default");
    assert_eq!(project.theme.accent_color, "#FF6B35");
    assert_eq!(
        project.theme.background_color,
        Some("#1A1A2E".to_string())
    );
    assert_eq!(
        project.theme.custom_vars.get("hero_text"),
        Some(&"Welcome to My Awesome App".to_string())
    );

    // Files
    assert_eq!(project.files.len(), 2);
    assert_eq!(project.files[0].source, "dist/");
    assert!(project.files[0].recursive);

    // Steps
    assert_eq!(project.steps.len(), 4);

    // Check extract step
    match &project.steps[0] {
        InstallStep::Extract { label } => {
            assert_eq!(label, "Installing application files...");
        }
        _ => panic!("Expected Extract step"),
    }

    // Check shortcut step
    match &project.steps[1] {
        InstallStep::Shortcut {
            label,
            target,
            locations,
            name,
        } => {
            assert_eq!(label, "Creating desktop shortcut...");
            assert_eq!(target, "{install_dir}/myapp.exe");
            assert!(locations.contains(&ShortcutLocation::Desktop));
            assert!(locations.contains(&ShortcutLocation::StartMenu));
            assert_eq!(name, &Some("My Awesome App".to_string()));
        }
        _ => panic!("Expected Shortcut step"),
    }

    // Check registry step
    match &project.steps[2] {
        InstallStep::Registry {
            hive,
            key,
            value_name,
            value_data,
            ..
        } => {
            assert_eq!(hive, "HKEY_LOCAL_MACHINE");
            assert_eq!(key, "SOFTWARE\\AcmeCorp\\MyAwesomeApp");
            assert_eq!(value_name, "InstallPath");
            assert_eq!(value_data, "{install_dir}");
        }
        _ => panic!("Expected Registry step"),
    }

    // Prerequisites
    assert_eq!(project.prerequisites.len(), 2);
    match &project.prerequisites[1] {
        Prerequisite::DotNet {
            minimum_version, ..
        } => {
            assert_eq!(minimum_version, "6.0");
        }
        _ => panic!("Expected DotNet prerequisite"),
    }

    // Uninstaller
    let uninstall = project
        .uninstall
        .as_ref()
        .expect("Uninstall config should exist");
    assert!(uninstall.enabled);
    assert_eq!(uninstall.exe_name, "uninstall.exe");
}

#[test]
fn test_roundtrip_serialize_deserialize() {
    let toml = sample_project_toml();
    let project = LynxProject::from_toml_str(&toml).expect("Should parse");
    let serialized = project.to_toml_str().expect("Should serialize");
    let reparsed = LynxProject::from_toml_str(&serialized).expect("Should re-parse");

    assert_eq!(reparsed.app.name, project.app.name);
    assert_eq!(reparsed.app.version, project.app.version);
    assert_eq!(reparsed.steps.len(), project.steps.len());
    assert_eq!(reparsed.files.len(), project.files.len());
}

#[test]
fn test_missing_required_field_fails() {
    let bad_toml = [
        "[app]",
        "name = \"Incomplete App\"",
        "",
        "[theme]",
        "name = \"default\"",
        "path = \"themes/default\"",
    ]
    .join("\n");

    let result = LynxProject::from_toml_str(&bad_toml);
    assert!(result.is_err(), "Should fail with missing required fields");
}

#[test]
fn test_minimal_valid_project() {
    let minimal = [
        "[app]",
        "name = \"Tiny App\"",
        "version = \"1.0.0\"",
        "publisher = \"Me\"",
        "id = \"com.me.tinyapp\"",
        "",
        "[theme]",
        "name = \"default\"",
        "path = \"themes/default\"",
    ]
    .join("\n");

    let project = LynxProject::from_toml_str(&minimal).expect("Minimal project should parse");
    assert_eq!(project.files.len(), 0);
    assert_eq!(project.steps.len(), 0);
    assert_eq!(project.prerequisites.len(), 0);
    assert!(project.uninstall.is_none());
    assert_eq!(project.theme.accent_color, "#FF6B35");
}