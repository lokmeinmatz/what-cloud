import { store } from '@/store'
import { Folder, pathArrayToString } from './fs'
import { delay } from './utils';


// TODO handle reject case
export interface UploadStatus {
    totalFiles: number;
    currentFileNum: number;
    currentFile: string;
    percent: number;
    res: (() => void) | null; 
    rej: (() => void) | null; 

}

export function nextFile(status: UploadStatus, name: string) {
    console.log('next file', name, status)
    status.currentFileNum++
    status.currentFile = name
}

export function finished(status: UploadStatus) {
    status.currentFileNum = -1
    status.currentFile = "Finished upload"
    status.res?.()
}

export function waitForFinish(status: UploadStatus): Promise<void> {
    return new Promise((res, rej) => {
        status.res = res
        status.rej = rej
    })
}


// returns a status object that tracks the upload process
// expects uploadStatus to be reactive
export function uploadFiles(uploadStatus: UploadStatus, root: Folder, files: FileList): UploadStatus {
    uploadStatus.currentFile = 'starting upload...'
    uploadStatus.currentFileNum = 0
    uploadStatus.res = null
    uploadStatus.rej = null
    uploadStatus.percent = 0
    uploadStatus.totalFiles = files.length


    const upload = async () => {
        for (const file of files) {
            //await delay(1000 * 100)
            

            // async version of xml request
            const asyncReq: () => Promise<void> = () => {
                const xhr = new XMLHttpRequest()
                xhr.upload.onprogress = e => {
                    uploadStatus.percent = e.loaded / e.total
                }
                const url = `/api/upload?file_path=${encodeURIComponent(pathArrayToString([...root.pathFromRoot, file.name]))}`
                console.log('Uploading file to', url)
                xhr.open('POST', url)
                xhr.setRequestHeader('Authorization', `Bearer ${store.user.value?.authToken}`)
                nextFile(uploadStatus, file.name)
                xhr.onerror = console.error
                
                return new Promise((res, rej) => {
                    xhr.onload = () => res()
                    
                    xhr.onerror = xhr.onabort = () => rej()
                    xhr.onreadystatechange = console.log
                    xhr.send(file)
                })
            }


            await asyncReq()
            console.log('finished upload')
        }

        // refetch data
        root.fetched = false
        root.fetch()
        finished(uploadStatus)
    }
    console.log('calling upload()')
    setTimeout(upload, 0)

    console.log('returning uploadStatus')

    return uploadStatus
}