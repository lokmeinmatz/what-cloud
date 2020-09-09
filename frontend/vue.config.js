module.exports = {
    devServer: {
        proxy:  {
            '/api': {
                //logLevel: 'debug',
                target: 'http://[::1]:8000'
            }    
        }
    }
}