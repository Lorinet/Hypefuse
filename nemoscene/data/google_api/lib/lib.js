class GoogleAuth {
    constructor(client_id) {
        this.client_id = client_id;
    }

    getLoginCode() {
        return GoogleAuth.#postRequest("https://oauth2.googleapis.com/device/code",  {
            client_id: this.client_id,

        })
    }

    static #postRequest(url, params) {
        let request = new XMLHttpRequest();
        request.open("POST", url, false);
        request.setRequestHeader('Content-type', 'application/x-www-form-urlencoded');
        request.send(params);
        if(request.status === 200) {
            return JSON.parse(request.responseText);
        } else {
            return false;
        }
    }
}