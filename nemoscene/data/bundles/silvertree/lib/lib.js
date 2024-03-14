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
        request.open("GET", this.serverAddress + "/config?uuid=" + this.uuid + "&base=" + base + "&key=" + key, false);
        request.send(null);
        if(request.status === 200) {
            return JSON.parse(request.responseText);
        } else {
            return false;
        }
    }
}