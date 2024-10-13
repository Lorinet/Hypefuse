class Silvertree {
    static uuid = "";
    static serverAddress = "http://localhost:1337";

    static initFramework(uuid, external = false) {
        if(external) {
            if(!window.location.href.includes("localhost")) {
                this.serverAddress = "http://linfinitysmartmirror.local:1337";
            }
        }

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

    static requestSync(method, url, body) {
        let request = new XMLHttpRequest();
        request.open("GET", url, false);
        if(method === "POST") {
            request.setRequestHeader('Content-Type', 'application/x-www-form-urlencoded');
        }
        request.send(body);
        if(request.status === 200) {
            return request.responseText;
        } else {
            document.getElementById("err").innerText += request.statusText + request.responseText;
            return null;
        }
    }

    static getConfigurationValue(base, key) {
        let response = this.requestSync("GET", this.serverAddress + "/config_get?uuid=" + encodeURIComponent(this.uuid) + "&base=" + encodeURIComponent(base) + "&key=" + encodeURIComponent(key), null);
        if (response !== null) {
            return JSON.parse(response);
        } else {
            return false;
        }
    }

    static getConfigurationBase(base) {
        let response = this.requestSync("GET", this.serverAddress + "/config_get_base?uuid=" + encodeURIComponent(this.uuid) + "&base=" + encodeURIComponent(base), null);
        if (response !== null) {
            return JSON.parse(response);
        } else {
            return false;
        }
    }

    static setConfigurationValue(base, key, value) {
        return this.setGlobalConfigurationValue(this.uuid, base, key, value);
    }

    static setGlobalConfigurationValue(bundle, base, key, value) {
        let response = this.requestSync("GET", this.serverAddress + "/config_set?uuid=" + encodeURIComponent(bundle) + "&base=" + encodeURIComponent(base) + "&key=" + encodeURIComponent(key) + "&value=" + encodeURIComponent(value), null);
        return response !== null;
    }

    static getGlobalConfiguration() {
        let response = this.requestSync("GET", this.serverAddress + "/config_all", null);
        if (response !== null) {
            return JSON.parse(response);
        } else {
            return false;
        }
    }

    static createGlobalConfigurationBase(bundle, base) {
        let response = this.requestSync("GET", this.serverAddress + "/config_create_base?uuid=" + encodeURIComponent(bundle) + "&base=" + encodeURIComponent(base), null);
        return response !== null;
    }

    static deleteGlobalConfigurationBase(bundle, base) {
        let response = this.requestSync("GET", this.serverAddress + "/config_delete_base?uuid=" + encodeURIComponent(bundle) + "&base=" + encodeURIComponent(base), null);
        return response !== null;
    }

    static reloadDashboard() {
        this.requestSync("GET", this.serverAddress + "/trigger_reload_dashboard", null);
    }

    static reloadSystem() {
        this.requestSync("GET", this.serverAddress + "/trigger_reload_system", null);
    }

    static checkSystemPassword(password) {
        let response = this.requestSync("POST", this.serverAddress + "/authenticate", `password=${encodeURIComponent(password)}`, null);
        if (response !== null) {
            return JSON.parse(response);
        } else {
            return false;
        }
    }

    static reconnectNetwork() {
        this.requestSync("GET", this.serverAddress + "/trigger_reconnect_network", null);
    }

    static proxyGet(url) {
        return this.requestSync("GET", this.serverAddress + "/proxy?url=" + encodeURIComponent(url), null);
    }

    static getBundlePath() {
        return this.serverAddress + "/bundle/" + this.uuid;
    }


}