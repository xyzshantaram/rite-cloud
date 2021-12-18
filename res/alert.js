const ALERT_EMPTY_FIELD = 0;
const ALERT_CANCELLED = 1;

function alert_CreateElement(parent, tag, className, id, innerHTML, misc) {
    let elem = document.createElement(tag);
    if (className) elem.className = className;
    if (id) elem.id = id;
    if (innerHTML) elem.innerHTML = innerHTML;
    if (misc) {
        for (let x in misc) {
            elem[x] = misc[x];
        }
    }
    parent.appendChild(elem);
    return elem;
}

function initialiseAlert() {
    let mask = alert_CreateElement(document.body, 'div', 'v-flex-centered', 'alert-mask');
    let alert = alert_CreateElement(mask, 'div', 'v-flex-centered', 'alert');
    let alertH = alert_CreateElement(alert, 'div', '', 'alert-header');
    let alertB = alert_CreateElement(alert, 'div', '', 'alert-body');
    let alertF = alert_CreateElement(alert, 'div', '', 'alert-footer');
    let alertA = alert_CreateElement(alertF, 'button', '', 'alert-action', 'Action');
    let alertC = alert_CreateElement(alertF, 'button', '', 'alert-close', 'Close');
    alertC.onclick = hideAlert();
}

function createAlert(title, body, mode, callback, callbackLabel) {
    const mask = document.getElementById('alert-mask');
    const alert = document.getElementById('alert');
    // setting mask's display to flex makes every child visible
    mask.style.display = 'flex';
    mask.style.animation = '0.1s fade-out reverse';
    // close button is only hidden in case of an error dialog, so we set this to block by default

    document.getElementById('alert-header').innerHTML = title || "Alert"; // set dialog title
    document.getElementById('alert-body').innerHTML = body; // set dialog body

    let footer = document.getElementById('alert-footer');
    footer.innerHTML = '';
    let action = alert_CreateElement(footer, 'button', '', 'alert-action', 'Action');
    let close = alert_CreateElement(footer, 'button', '', 'alert-close', 'Close');

    close.style.display = 'block';
    if (mode === 'error') {
        close.style.display = 'none'; // no point letting user continue on a broken site
        if (!callback) {
            // if no callback is supplied, this means there's no action and thus no buttons are present.
            // in such a case, we hide the footer
            document.getElementById('alert-footer').style.display = 'none';
        }
    } else {
        close.onclick = hideAlert;
    }

    if (callback) { // if callback exists, enable the button for it and set the click action
        action.style.display = 'block';
        action.innerHTML = callbackLabel;
        action.onclick = () => {
            callback();
        }
    } else {
        action.style.display = 'none';
    }
}

function hideAlert() {
    document.getElementById('alert-mask').style.display = 'none';
}

function createPrompt(title, msg, callback, callbackLabel, onErr) {
    const mask = document.getElementById('alert-mask');
    const alert = document.getElementById('alert');
    const body = document.getElementById('alert-body');
    const wrapper = document.createElement('div');
    document.getElementById('alert-header').innerHTML = title || "Input"; // set dialog title

    body.innerHTML = '';
    let footer = document.getElementById('alert-footer');
    footer.innerHTML = '';
    let action = alert_CreateElement(footer, 'button', '', 'alert-action', 'Action');
    let close = alert_CreateElement(footer, 'button', '', 'alert-close', 'Close');

    // setting mask's display to flex makes every child visible
    mask.style.display = 'flex';
    mask.style.animation = '0.1s fade-out reverse';

    let txt = document.createTextNode(msg);

    wrapper.appendChild(txt);

    let field = document.createElement('input');
    field.type = 'text';

    wrapper.appendChild(field);
    body.appendChild(wrapper);

    action.style.display = 'block';
    action.innerHTML = callbackLabel || 'Submit';
    action.onclick = () => {
        if (field.value) {
            callback(field.value);
            hideAlert();
        } else {
            if (onErr) onErr(ALERT_EMPTY_FIELD);
            else hideAlert();
        }
    }

    close.onclick = () => {
        if (onErr) onErr(ALERT_CANCELLED);
        else hideAlert();
    }
}

function createConfirm(title, msg, yes, no, yesLabel, noLabel) {
    const mask = document.getElementById('alert-mask');
    const alert = document.getElementById('alert');
    document.getElementById('alert-header').innerHTML = title; // set dialog title
    const body = document.getElementById('alert-body');
    document.getElementById('alert-header').innerHTML = title || "Confirm"; // set dialog title
    body.innerHTML = '';

    // setting mask's display to flex makes every child visible
    mask.style.display = 'flex';
    mask.style.animation = '0.1s fade-out reverse';

    body.innerHTML = msg || 'Are you sure?';

    let yesBtn = document.createElement('button');
    yesBtn.innerHTML = yesLabel || "Yes";
    yesBtn.onclick = () => {
        if (yes) yes();
        hideAlert();
    }
    let noBtn = document.createElement('button');
    noBtn.innerHTML = noLabel || "No";
    noBtn.onclick = () => {
        if (no) no();
        hideAlert();
    }
    let footer = document.getElementById('alert-footer');
    footer.innerHTML = '';
    footer.append(yesBtn, noBtn);
}

function createPromisedAlert(title, message, label = 'OK') {
    return new Promise((resolve, reject) => {
        try {
            createAlert(title, message, 'error', () => {
                resolve(true);
            }, label)
        }
        catch (e) {
            reject(e);
        }
    })
}

function createPromisedPrompt(title, message, label) {
    return new Promise((resolve, reject) => {
        createPrompt(
            title,
            message,
            (val) => resolve(val),
            label,
            (e) => reject(e)
        );
    })
}

function createPromisedConfirm(title, message, yesLabel, noLabel) {
    return new Promise((resolve, reject) => {
        createConfirm(
            title,
            message,
            () => resolve(true),
            () => resolve(false),
            yesLabel,
            noLabel
        );
    })
}

window.alertjs = {
    message: createAlert,
    prompt: createPrompt,
    confirm: createConfirm,
    promises: {
        message: createPromisedAlert,
        prompt: createPromisedPrompt,
        confirm: createPromisedConfirm,
    }
}