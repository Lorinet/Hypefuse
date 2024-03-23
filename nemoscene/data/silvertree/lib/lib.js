class Silvertree {
    static uuid = "";
    static serverAddress = "http://localhost:1337";

    static initFramework(uuid) {
        const cssId = 'silvertree_app_style';
        if (!document.getElementById(cssId))
        {
            let head  = document.getElementsByTagName('head')[0];
            let link  = document.createElement('link');
            link.id = cssId;
            link.rel = 'stylesheet';
            link.type = 'text/css';
            link.href = this.serverAddress + '/bundle/silvertree/lib/silvertree_app.css';
            head.appendChild(link);
        }
        this.uuid = uuid;
    }

    static getConfigurationValue(base, key) {
        let request = new XMLHttpRequest();
        request.open("GET", this.serverAddress + "/config_get?uuid=" + encodeURIComponent(this.uuid) + "&base=" + encodeURIComponent(base) + "&key=" + encodeURIComponent(key), false);
        request.send(null);
        if(request.status === 200) {
            return JSON.parse(request.responseText);
        } else {
            return false;
        }
    }

    static setConfigurationValue(base, key, value) {
        return this.setGlobalConfigurationValue(this.uuid, base, key, value);
    }

    static setGlobalConfigurationValue(bundle, base, key, value) {
        let request = new XMLHttpRequest();
        request.open("GET", this.serverAddress + "/config_set?uuid=" + encodeURIComponent(bundle) + "&base=" + encodeURIComponent(base) + "&key=" + encodeURIComponent(key) + "&value=" + encodeURIComponent(value), false);
        request.send(null);
        if(request.status === 200) {
            return true;
        } else {
            return false;
        }
    }

    static getGlobalConfiguration() {
        let request = new XMLHttpRequest();
        request.open("GET", this.serverAddress + "/config_all", false);
        request.send(null);
        if(request.status === 200) {
            return JSON.parse(request.responseText);
        } else {
            return false;
        }
    }

    static createGlobalConfigurationBase(bundle, base) {
        let request = new XMLHttpRequest();
        request.open("GET", this.serverAddress + "/config_create_base?uuid=" + encodeURIComponent(bundle) + "&base=" + encodeURIComponent(base), false);
        request.send(null);
        if(request.status === 200) {
            return true;
        } else {
            return false;
        }
    }

    static deleteGlobalConfigurationBase(bundle, base) {
        let request = new XMLHttpRequest();
        request.open("GET", this.serverAddress + "/config_delete_base?uuid=" + encodeURIComponent(bundle) + "&base=" + encodeURIComponent(base), false);
        request.send(null);
        if(request.status === 200) {
            return true;
        } else {
            return false;
        }
    }

    static reloadDashboard() {
        let request = new XMLHttpRequest();
        request.open("GET", this.serverAddress + "/trigger_reload_dashboard", false);
        request.send(null);
    }

    static reloadSystem() {
        let request = new XMLHttpRequest();
        request.open("GET", this.serverAddress + "/trigger_reload_system", false);
        request.send(null);
    }
}