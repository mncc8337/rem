# rem
a minimal CLI interval reminder. i create this to remind myself to drink enough water and rest my eyes.
## build
just `cargo build --release`  
## usage
for more see `rem -h` or `rem <command> -h`  
default config location is `~/.config/rem.json`, you can use other location by adding `-c <loc>` after `rem` for every command for example  
```
rem -c <loc> start
rem -c <loc> add <name> <int>
```
### start the reminder process
```rem start```
keep this process running  
after that you can run `rem add bla bla` or `rem remove bla bla` separately  
### add a reminder
```rem add <name> <interval>```
### remove a reminder
```rem remove <index>```
get the index by running `rem list`
### list all reminders
```rem list```
use `-v` for verbose output
