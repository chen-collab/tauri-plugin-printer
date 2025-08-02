# Tauri Plugin printer

Repositories which I forked from:

[Alfian Lensun's Original Plugin Repository](https://github.com/alfianlensundev/tauri-plugin-printer)

[adao99's Fork for Tauri V2 Beta](https://github.com/adao99/tauri-plugin-printer-v2)

Tauri V2 stable requires a different cofiguration with newly added features like permissions, capabilities which makes the code in the previous repos backwards-incompatible.

Development Worflow:

### 1. Created Plugin Scaffold 
```bash
npx @tauri-apps/cli plugin new printer
```

### 2. Copied the code from the old repo to this project

### 3. Setup permissions
Add allow clauses for **ALL** commands in ```permissions/default.toml```

```toml
[default]
description = "Default permissions for the plugin"
permissions = ["allow-create-temp-file", "allow-remove-temp-file", "allow-get-printers", "allow-get-printers-by-name", "allow-print-pdf", "allow-get-jobs", "allow-get-jobs-by-id", "allow-resume-job", "allow-restart-job", "allow-pause-job", "allow-remove-job"]
```
⚠️ There might be a better way to do this (I tried using `build.rs` it did not set default permissions, it set for each command separately)


### 4. Add plugin to your project
- If you are using the plugin locally, you have to add `printer:default` to your `capabilities/default.json` under permissions

    ``` json
    {
        "permissions": [
            "printer:default"
        ]
    }
  ```
- If you use `npx tauri add`, you dont need to. (I think so, didn't check.)