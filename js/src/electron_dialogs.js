const { dialog } = require('electron');


function messageBox(value, cb) {
    // TODO: default button
    // TODO: icon...
    switch (value.box_type) {
        case 'Ok':
            dialog.showMessageBox({
                'title': value.title,
                'message': value.message,
                'buttons': ['Ok']
            }).then(result => {
                cb('Ok');
            }).catch(err => console.log(err));
            break;
        case 'OkCancel':
            dialog.showMessageBox({
                'title': value.title,
                'message': value.message,
                'buttons': ['Ok', 'Cancel']
            }).then(result => {
                switch (result.response) {
                    case 0:
                        cb('Ok');
                        break;
                    case 1:
                        cb('Cancel');
                        break;
                }
            }).catch(err => console.log(err));
            break;
        case 'YesNo':
            dialog.showMessageBox({
                'title': value.title,
                'message': value.message,
                'buttons': ['Yes', 'No']
            }).then(result => {
                switch (result.response) {
                    case 0:
                        cb('Yes');
                        break;
                    case 1:
                        cb('No');
                        break;
                }
            }).catch(err => console.log(err));
            break;
    };
}

function handleFileSave(value, cb) {
    dialog.showSaveDialog({
        'title': value.title,
        'defaultPath': value.path,
        'filters': value.filter,
    }).then(result => {
        if (result.canceled) {
            cb('Cancel');
            return;
        }
        cb({'SaveTo': result.filePath});
    }).catch(err => console.log(err));
}

function handleFileOpen(value, cb) {
    let props = [];
    if (value.multiple) {
        props.push('multiSelections');
    }
    dialog.showOpenDialog({
        'title': value.title,
        'filters': value.filter,
        'properties': props,
        'defaultPath': value.path
    }).then(result => {
        if (result.canceled) {
            cb('Canceled');
            return;
        }
        if (value.multiple) {
            cb({'Selected': result.filePaths[0]})
        } else {
            cb({'SelectedMultiple': result.filePaths})
        }
    }).catch(err => console.log(err));
}


export default function showDialog(dialog, cb) {
    let type = dialog['__type__'];
    switch(type) {
        case "MessageBox":
            messageBox(dialog, cb);
            break;
        case "FileSaveDialog":
            handleFileSave(dialog, cb);
            break;
        case "FileOpenDialog":
            handleFileOpen(dialog, cb);
            break;
    }
}