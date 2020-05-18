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
    console.log(value);
    let filters = [];
    if (value.filters) {
        for (const filter of value.filters) {
            filters.push({'name': filter.description, 'extensions': filter.filters});
        }
    }    
    dialog.showSaveDialog({
        'title': value.title,
        'defaultPath': value.path,
        'filters': filters,
    }).then(result => {
        if (result.canceled) {
            cb('Cancel');
            return;
        }
        cb({'SaveTo': value.filePath});
    }).catch(err => console.log(err));
}

function handleFileOpen(value, cb) {
    let props = [];
    if (value.multiple) {
        props.push('multiSelections');
    }
    let filters = [];
    for (const filter of value.filters) {
        filters.push({'name': filter.description, 'extensions': filter.filters});
    }
    dialog.showOpenDialog({
        'title': value.title,
        'filters': filters,
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