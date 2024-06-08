let uid = 0;
let godMode = false;
let password = null;

function getUID() {
    return uid++;
}

function peekaboo(id) {
    let vis = document.getElementById(id).style.display;
    if (vis === "none") {
        peekashow(document.getElementById(id));
    } else {
        document.getElementById(id).style.display = "none";
    }
}

function peekashow(element) {
    element.style.display = "block";
}

function addDropdown(element, title, addButton, deleteButton, addBase, content) {
    let uniq = getUID();
    let idHeader = `dropdown_${uniq}`;
    let idContent = `dropdown_content_${uniq}`;

    let headerTitle = document.createElement("div");
    headerTitle.classList.add("dropdown_title");
    headerTitle.innerText = title;

    let headerButtons = document.createElement("div");
    headerButtons.classList.add("dropdown_button_area");
    if (addBase) {
        addButton = function (event) {
            event.stopPropagation();
            peekashow(document.getElementById(idContent));
            addConfigurationWizard(document.getElementById(idContent), title);
        };
    }
    if (addButton != null) {
        let button = document.createElement("i");
        button.classList.add("fa-solid");
        button.classList.add("fa-plus");
        button.classList.add("dropdown_button");
        button.onclick = addButton;
        headerButtons.appendChild(button);
    }
    if (deleteButton != null) {
        let button = document.createElement("i");
        button.classList.add("fa-solid");
        button.classList.add("fa-trash");
        button.classList.add("dropdown_button");
        button.onclick = deleteButton;
        headerButtons.appendChild(button);
    }

    let header = document.createElement("div");
    header.id = idHeader;
    header.classList.add("dropdown_header");
    header.appendChild(headerTitle);
    header.appendChild(headerButtons);
    header.onclick = function () {
        peekaboo(idContent)
    };

    let contentBox = document.createElement("div");
    contentBox.id = idContent;
    contentBox.style.display = "none";
    contentBox.classList.add("dropdown_container");
    contentBox.appendChild(content);

    element.appendChild(header);
    element.appendChild(contentBox);

    return idContent;
}

function addSetting(table, keyUID, key, value) {
    let keyCell = document.createElement("td");
    let keyCellInput = document.createElement("input");
    keyCellInput.type = "text";
    keyCellInput.name = `key_${keyUID}`;
    if (key != null) {
        keyCellInput.value = key;
    }
    keyCellInput.classList.add("setting_box");
    keyCellInput.classList.add("setting_key");
    keyCellInput.placeholder = "type here...";
    if (key != null) {
        keyCellInput.readOnly = true;
    }
    keyCell.appendChild(keyCellInput);
    let valueCell = document.createElement("td");
    let valueCellInput = document.createElement("input");
    if (key.startsWith("password")) {
        valueCellInput.type = "password";
    } else {
        valueCellInput.type = "text";
    }
    valueCellInput.name = `value_${keyUID}`;
    if (value != null) {
        valueCellInput.value = value;
    }
    valueCellInput.placeholder = "type here...";
    valueCellInput.classList.add("setting_box");
    valueCell.appendChild(valueCellInput);

    let row = document.createElement("tr");
    row.appendChild(keyCell);
    row.appendChild(valueCell);
    table.appendChild(row);
}

function addSettingsEditor(element, configuration, bundle, baseName) {
    let uid = getUID();
    let formID = `form_${uid}`;
    let tableID = `table_${uid}`;
    let form = document.createElement("form");
    form.id = formID;
    let table = document.createElement("table");
    table.id = tableID;
    table.classList.add("settings_table");
    let keyUID = 0;
    Object.keys(configuration).forEach(function (key) {
        addSetting(table, keyUID, key, configuration[key]);
        keyUID++;
    });
    form.appendChild(table);

    let saveButton = document.createElement("div");
    saveButton.classList.add("save_button");
    saveButton.innerText = "Save";
    saveButton.onclick = function () {
        let base = Object.fromEntries(new FormData(document.getElementById(formID)));
        let count = Object.keys(base).length / 2;
        for (let i = 0; i < count; i++) {
            console.log([bundle, baseName, base[`key_${i}`], base[`value_${i}`]]);
            let val = base[`value_${i}`];
            if (val === "true") {
                val = true;
            } else if (val === "false") {
                val = false;
            } else if (isNumeric(val)) {
                val = parseInt(val);
            }
            Silvertree.setGlobalConfigurationValue(bundle, baseName, base[`key_${i}`], JSON.stringify(val));
        }
        if(bundle === "wifi") {
            Silvertree.reconnectNetwork();
            Silvertree.reloadSystem();
        }
        Silvertree.reloadDashboard();
        showConfiguration();
    };
    form.appendChild(saveButton);

    element.appendChild(form);
    return tableID;
}

function isUserCustomizable(bundle) {
    return bundle === "widgets" || bundle === "wifi";
}

function showLogin(element) {
    let title = document.createElement("h3");
    title.classList.add("password_title");
    title.innerText = "Enter password to unlock";

    let passwordInput = document.createElement("input");
    passwordInput.type = "password";
    passwordInput.placeholder = "password...";
    passwordInput.id = "password_box";
    passwordInput.classList.add("password_box");

    let loginButton = document.createElement("div");
    loginButton.classList.add("save_button");
    loginButton.innerText = "Unlock";
    loginButton.onclick = function() {
        let pval = document.getElementById("password_box").value;
        let ok = Silvertree.checkSystemPassword(pval);
        if(ok) {
            password = pval;
            showConfiguration();
        }
    }

    element.appendChild(title);
    element.appendChild(passwordInput);
    element.appendChild(loginButton);
}

function renderConfiguration(element, configuration) {
    element.innerHTML = "";
    Object.keys(configuration).forEach(function (bundle_config) {
        let bases = document.createElement("div");
        let lastBase = "";
        Object.keys(configuration[bundle_config]).forEach(function (base) {
            if (base === "bundle" && !godMode) {
                return;
            }
            lastBase = base;
            let settingsContainer = document.createElement("div");
            let settingsTableID = addSettingsEditor(settingsContainer, configuration[bundle_config][base], bundle_config, base);
            let addAction = null;
            if (godMode) {
                addAction = function (event) {
                    event.stopPropagation();
                    let table = document.getElementById(settingsTableID);
                    let form = table.parentElement;
                    let keyUID = Object.keys(Object.fromEntries(new FormData(form))).length / 2;
                    addSetting(table, keyUID, null, null);
                };
            }
            let deleteAction = null;
            if (isUserCustomizable(bundle_config) || godMode) {
                deleteAction = function () {
                    Silvertree.deleteGlobalConfigurationBase(bundle_config, base);
                    Silvertree.reloadDashboard();
                    if(isUserCustomizable(bundle_config)) {
                        if (bundle_config === "wifi") {
                            Silvertree.reconnectNetwork();
                        }
                        Silvertree.reloadSystem();
                    }
                    showConfiguration();
                }
            }
            addDropdown(bases, base, addAction, deleteAction, false, settingsContainer);
        });
        let addAction = false;
        if (isUserCustomizable(bundle_config) || godMode) {
            addAction = true;
        }
        if (bases.children.length > 0 || godMode) {
            if (bases.children.length > 2 || godMode || isUserCustomizable(bundle_config) || lastBase !== bundle_config) {
                addDropdown(element, bundle_config, null, null, addAction, bases);
            } else if (bases.children.length === 2) {
                element.appendChild(bases.getElementsByTagName("div")[0]);
                element.appendChild(bases.getElementsByTagName("div")[0]);
            }
        }
    });
}

function addConfigurationWizard(element, template) {
    let uniq = getUID();
    let idHeader = `wizard_${uniq}`;
    let idHeaderNameInput = `wizard_input_${uniq}`;

    let headerTitle = document.createElement("input");
    headerTitle.id = idHeaderNameInput;
    headerTitle.type = "text";
    headerTitle.placeholder = "enter name...";
    headerTitle.classList.add("dropdown_title");

    let headerButtons = document.createElement("div");
    headerButtons.classList.add("dropdown_button_area");
    let button = document.createElement("i");
    button.classList.add("fa-solid");
    button.classList.add("fa-plus");
    button.classList.add("dropdown_button");
    button.onclick = function () {
        let base = document.getElementById(idHeaderNameInput).value;
        Silvertree.createGlobalConfigurationBase(template, base);
        if (template === "widgets") {
            Silvertree.setGlobalConfigurationValue(template, base, "position_x", 100);
            Silvertree.setGlobalConfigurationValue(template, base, "position_y", 100);
            Silvertree.setGlobalConfigurationValue(template, base, "width", 100);
            Silvertree.setGlobalConfigurationValue(template, base, "height", 100);
            Silvertree.setGlobalConfigurationValue(template, base, "uuid", JSON.stringify("clock"));
        } else if (template === "wifi") {
            Silvertree.setGlobalConfigurationValue(template, base, "name", JSON.stringify(base));
            Silvertree.setGlobalConfigurationValue(template, base, "password", JSON.stringify(""));
        }
        showConfiguration();
    };
    headerButtons.appendChild(button);

    let header = document.createElement("div");
    header.id = idHeader;
    header.classList.add("dropdown_header");
    header.appendChild(headerTitle);
    header.appendChild(headerButtons);

    element.appendChild(header);
}

function showConfiguration() {
    if(password == null) {
        showLogin(document.getElementById("settings_container"));
    } else {
        renderConfiguration(document.getElementById("settings_container"), Silvertree.getGlobalConfiguration());
    }
}

let godCounter = 0;

function godBump() {
    godCounter++;
    if(godCounter >= 10) {
        god();
    }
}

function god() {
    godMode = true;
    document.getElementById("admin_mode_label").style.display = "block";
    showConfiguration();
}

function isNumeric(str) {
    if (typeof str != "string") {
        return false;
    }
    return !isNaN(str) &&
        !isNaN(parseFloat(str));
}