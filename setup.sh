#!/bin/bash
echo Compiling with cargo...
cargo build --release
echo Copying to $HOME/.local/bin
cp target/release/fsh $HOME/.local/bin/

if [ ! -f $HOME/.config/fish/functions/fish_prompt.fish ]; then
    echo "Creating $HOME/.config/fish/functions/fish_prompt.fish"
    echo "function fish_prompt
    set FSH_LAST_STATUS \$status
    fsh \$FSH_LAST_STATUS
end" > $HOME/.config/fish/functions/fish_prompt.fish
fi
