Output cli.gif
Set FontSize 24
Set Width 1920
Set Height 1080
Set Theme "GruvboxDark"

Set Shell zsh
Type@50ms "bmm search rust"
Enter
Sleep 2000ms
Type "clear"
Enter
Type@50ms "bmm list --uri github.com --title dark --tags vim-colorschemes -f delimited"
Enter
Sleep 3000ms
Type "clear"
Enter
Type@30ms "bmm save https://github.com/dhth/omm --title 'manage tasks via the CLI' --tags tools,productivity"
Enter
Sleep 1000ms
Type "clear"
Enter
Type@50ms "bmm search 'manage tasks' -f json"
Enter
Sleep 2000ms
