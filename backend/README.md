# backend

## GET /api/static/icons/<ext/"folder">

should return generated icon

## POST /api/user/login

payload: {name: string, password_base64: string}

returns {name: string, profile_picture_url: string, auth_token: string}

## GET /folder?url_encoded_path=...

get non-recursive Folder data

{
    name: string,
    childrenFolder: string[],
    files: string[],
    pathFromRoot: string[]
}