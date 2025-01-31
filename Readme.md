# Metadata
 - version: v0.1.0-dev

# Structure
1. csv files seperate by date and data name
2. sqlite files seperate by date

# config
```config.json
{
    global: {
        log4rs_cfg: <log4rs config path>,
        serial_port: {
            path: <serial port device ident.>,
            baudrate: <serial port baudrate>
        },
        listen_list:[
            {
                name: <name>,
                path: <listen file path>,
                ftype: <listen file type>, // [file, pipe, ...]
                tag: <tag>,
                dkind: <dkind>,
                header: {
                    number: <number of file header line>
                },
                flags: {
                    f_move: <move listen file>
                }
            }, ...
        ],
    },
    codec: {
        <tag>: {
            tag: <tag>,
            <db_type>: { [rawdata, l1_data, sqlite3]
                directory: <path>,
                seperate_by: [optional] <file seperate method>,
                pattern: [optional] <filename pattern>,
                regex: [deprecated][optional] <filename pattern, used to figure out last modify file>,
                suffix: [optional] <file suffix, default: `dat`>
            },
            metadatas: [
                {
                    name: <data name>,
                    dkind: <data kind>,
                    raw_save: [optional] <save raw data into sqlite3>,
                    formation: [
                        {
                            spec: { // spec info
                                name: <name>,
                                description: <desc>,
                                dtype: <data type>,
                                unit: <data unit>,
                                float_number: [deprecated] <float number>
                            },
                            rust: { // rust system info
                                name: <name>,
                                dtype: <rust data type>,
                                unit: <rust data unit or format>,
                                major_datetime: [optional] <mark as major datetime>,
                            },
                            sqlite3: {
                                name: <name>,
                                dtype: <sqlite3 data type>,
                                unit: <sqlite3 data unit or format>
                            }
                        }
                    ]
                }, ...
            ]
        }
    } 
}

```

# Workflow
1. Receive data
2. dispatch (mpsc)
 - logger
    - system log: all
    - data log: ~7D
    - error log: all
 - csv data
 - sqlite data
 - database uploader


# TODO
1. Rework receiver register method

# ubuntu build dependence
```
apt install build-essential
sudo apt install -y pkg-config libusb-1.0-0-dev libftdi1-dev
sudo apt-get install -y  libudev-dev
```