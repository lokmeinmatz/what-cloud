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

For files, the size is in number of bytes. For folders,
currently we sum up all file sizes directly inside the folder (direct children).

In the future, it might be nice to cache the folder sizes so they get more accurate over time.
The question is, if this is possible on the backend without too much overhead.

## GET /api/user

Returns current user or null if not logged in

## GET /api/download/file?path=...&token=...

Download file, token is the auth token (maybe change to extra token in future?)