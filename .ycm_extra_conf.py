def Settings( **kwargs ):
    return {
            'ls': {
                "rust-analyzer": {
                     "diagnostics": { "disabled": ["incorrect-ident-case"] } 
                    }
                }
            }
