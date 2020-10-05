module.exports = {
    devServer: {
        proxy:  {
            '/api': {
                //logLevel: 'debug',
                target: 'http://127.0.0.1:8000'
            }    
        }
    }
}