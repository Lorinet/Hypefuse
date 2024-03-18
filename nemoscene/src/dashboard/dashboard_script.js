function reloadWhenRequested() {
    try {
        let request = new XMLHttpRequest();
        request.open("GET", "http://localhost:1337/reload_dashboard", false);
        request.send(null);
        if (request.status === 200) {
            if (JSON.parse(request.responseText)) {
                window.location.reload();
            }
        }
    } catch (e) {
        console.log(e);
    }
    setTimeout(reloadWhenRequested, 10000);
}

reloadWhenRequested();