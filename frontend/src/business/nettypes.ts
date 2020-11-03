export interface UserLogin {
    name: string,
    passwordBase64: string
}

export interface JWTPayload {
    userName: string,
    profilePictureUrl?: string,
    userId: string
}

export interface Metadata {
    type: string,
    size: number,
    lastModified: string,
    shared?: string,
}


export interface NetNode {
    name: string,
    childrenFolder?: string[],
    files?: string[],
    pathFromRoot: string[],
    metadata: Metadata,
    ownedBy: string
}