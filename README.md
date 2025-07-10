# rem
a minimal CLI interval reminder. i create this to remind myself to drink enough water and rest my eyes.
## build
just `cargo build --release`
## usage
for more see `rem -h` or `rem <command> -h`  
default config location is `~/.config/rem.json`, currently there is noway to change it.  
### start a reminder process
```rem start```
### add a reminder
```rem add <name> <interval>```
### list all reminders
```rem list```
add `-v` for verbose output
### remove a reminder
```rem remove <index>```
