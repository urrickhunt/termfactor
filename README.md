# termfactor
is your terminal sick, mid or sus?
test your terminal features & find out.

<img width="859" alt="wez2" src="https://github.com/user-attachments/assets/1f230930-94cf-4aee-805a-3a6c640b0ac2">

### Requirements

[fastfetch](https://github.com/fastfetch-cli/fastfetch) must be installed. termfactor uses [fastfetch](https://github.com/fastfetch-cli/fastfetch) for terminal identification.

https://github.com/fastfetch-cli/fastfetch

### Tests

termfactor tests your terminal for: 

- [truecolor](https://github.com/termstandard/colors)
- [sgr or text decorations](https://en.wikipedia.org/wiki/ANSI_escape_code#Select_Graphic_Rendition_parameters)
- unicode
- emojis
- bidi awareness
- [osc 8 hyperlinks](https://github.com/Alhadis/OSC8-Adoption/)
- osc 9 notifications
- osc 777 notifications
- [sixel graphics](https://www.arewesixelyet.com/)

please note that this is just a general test for bidirectional awareness & not a comprehensive bidi language test.

further bidi terminal resources:

- [BiDiSupport](https://gist.github.com/XVilka/a0e49e1c65370ba11c17)

- [BiDi in Terminal Emulators](https://terminal-wg.pages.freedesktop.org/bidi/)

### Installation

`cargo install termfactor`

### Building

`git clone https://github.com/urrickhunt/termfactor`

`cargo build --release`

`cargo install --path .`


### Track termfactors

<img width="930" alt="terms3" src="https://github.com/user-attachments/assets/c826255a-32af-4c98-a8b9-d9ddb5cf4178">
