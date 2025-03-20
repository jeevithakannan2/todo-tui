# Todo TUI

<!-- ![Preview](/.github/preview.gif) -->

> [!NOTE]
> Since the project is still in active development, you may encounter some issues. Please consider [submitting feedback](https://github.com/jeevithakannan2/todo-tui/issues) if you do.

## 💡 Usage
### Linux
```bash
curl -fLO https://github.com/jeevithakannan2/todo-tui/releases/latest/download/todo-tui && chmod +x todo-tui && ./todo-tui
```
### Windows
```powershell
Invoke-WebRequest -Uri "https://github.com/jeevithakannan2/todo-tui/releases/latest/download/todo-tui.exe" -OutFile "todo-tui.exe"
Start-Process -FilePath .\todo-tui.exe
```
### Configuration
TodoTUI supports TOML configuration. 

The configuration file can be found in `$HOME/.config/todotui/config.toml` for linux users

`C:\Users\user\AppData\Roaming\CodeTrenchers\TodoTUI\config\config.toml` for windows users

To turn on **Encryption** set
```toml
# config.toml
encryption = true
```

## 💖 Support

If you find **Todo TUI** interesting, please consider giving it a ⭐️ to show your support!
