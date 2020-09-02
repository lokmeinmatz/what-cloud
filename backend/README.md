# backend

## GET /api/static/icons/<ext/"folder">

should return generated icon

## POST /api/user/login

payload: {name: string, password_base64: string}

returns {name: string, profile_picture_url: string, auth_token: string}

## GET /api/folder?url_encoded_path=...

get non-recursive Folder data

{
    name: string,
    childrenFolder: string[],
    files: string[],
    pathFromRoot: string[]
}

## GET /api/metadata?url_encoded_path=...

get Folder or file Metadata

{
    type: "file" | "folder",
    size: number | -1,
    lastModified: iso-time-string
}


## GET /api/user

Returns current user or null if not logged in

## GET /api/download/file?path=...&token=...

Download file, token is the auth token (maybe change to extra token in future?)