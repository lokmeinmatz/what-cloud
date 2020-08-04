module.exports = {
    devServer: {
        proxy:  {
            '/api': {
                target: 'http://[::1]:8000'
            }    
        }
    }
}