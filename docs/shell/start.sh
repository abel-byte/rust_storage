CONFIG=config/0.toml nohup ./rust_storage >> log/0.log &
CONFIG=config/1.toml nohup ./rust_storage >> log/1.log &
CONFIG=config/2.toml nohup ./rust_storage >> log/2.log &
CONFIG=config/3.toml nohup ./rust_storage >> log/3.log &
CONFIG=config/4.toml nohup ./rust_storage >> log/4.log &
CONFIG=config/5.toml nohup ./rust_storage >> log/5.log &
tail -f log/*.log