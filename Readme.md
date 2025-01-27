# Metadata
 - version: dev

# Structure
1. csv data seperate by date
2. sqlite data seperate by date


# config
```config.json
{
    global: {
    },
    data: [ // in-order list
        {
            name: String,
            sqlite_name: String,
            datatype: {
                rust: String,
                sqlite: String,
            },
            regexp: String
        },...
    ]
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