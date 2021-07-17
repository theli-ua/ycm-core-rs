def Settings( **kwargs ):
    return { 'ls': {
                'checkOnSave' : { 'command' : 'clippy', 'allTargets': 'true'},
                'procMacro' : {
                    'enable': True
                },
                'cargo' : {
                    'loadOutDirsFromCheck': True,
                    'runBuildScripts': True
                    }
                },
        }


