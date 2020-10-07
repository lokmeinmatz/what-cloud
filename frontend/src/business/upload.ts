import { reactive } from 'vue'
import { pathArrayToString } from './fs'

class UploadStatus {
    percentage = 0
    currentFile?: string

}


// returns a status object that tracks the upload process
export function uploadFiles(root: string[], files: FileList): UploadStatus {
    const uploadStatus = reactive(new UploadStatus())
    const upload = async () => {
        for (const file of files) {
            const xhr = new XMLHttpRequest()
            xhr.upload.onprogress = e => {
                console.log('upload proress', e)
            }
            const url = `/api/upload?file_path=${encodeURIComponent(pathArrayToString([...root, file.name]))}`
            console.log('Uploading file to', url)
            xhr.open('POST', url, false)
            uploadStatus.currentFile = file.name
            xhr.send(file)
        }
    }
    upload()

    return uploadStatus
}