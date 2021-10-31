# backend

## building the rust app

If openssl was not found, you need to install it manually and export the OPENSSL_DIR path to the installation, e.g. C:\Program Files\OpenSSL-Win64 if you used the default on windows

## GET /api/static/icons/<ext/"folder">

should return generated icon

## POST /api/user/login

payload: {name: string, password_base64: string}

returns {name: string, profile_picture_url: string, auth_token: string}

## GET /api/node?path=...

get non-recursive Node (Folder / File) data

{
    name: string,
    childrenFolder: string[],
    files: string[],
    pathFromRoot: string[],
    ownedBy: string (UserID)
    metadata: {
        type: "file" | "folder",
        size: number (bytes),
        lastModified: string (iso á la yyyy-mm-ddThh:mm:ssZ)
        shared: null | string (sharedID)
    }
}

## GET /api/user

Returns current user or null if not logged in

## GET /api/download/file?path=...&token=...

Download file, token is the auth token (maybe change to extra token in future?)



# Environments variables

- DATA_PATH: where the root dir for user data is
    - default: "./test_data"
- DB_PATH: where the sqlite db is stored
    - default: "./database.sqlite"
- ICON_CONF: where icon conf json file is stored
    - default: "./icon-conf.json"