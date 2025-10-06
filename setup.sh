#!/usr/bin/env bash
echo Compiling with cargo...
cargo build --release
echo Copying to $HOME/.local/bin
mkdir -p $HOME/.local/bin/
cp target/release/fsh $HOME/.local/bin/

enable_hostname="0"

while getopts "h" FLAG
do
    case "${FLAG}" in
        h) enable_hostname="1";;
    esac
done

if [ ! -f $HOME/.config/fish/functions/fish_prompt.fish ]; then
    mkdir -p $HOME/.config/fish/functions/
    if [ $enable_hostname = "1" ]; then
        echo "Creating $HOME/.config/fish/functions/fish_prompt.fish"
        echo "function fish_prompt
    set FSH_LAST_STATUS \$status
    fsh \$FSH_LAST_STATUS
end" > $HOME/.config/fish/functions/fish_prompt.fish
    else
        echo "Creating $HOME/.config/fish/functions/fish_prompt.fish"
        echo "function fish_prompt
    set FSH_LAST_STATUS \$status
    if test "$SSH_CLIENT" != "" || test "$SSH_TTY" != "" | test "$SSH_CONNECTION" != ""
        set --erase FSH_NO_HOSTNAME
    else
        set -x FSH_NO_HOSTNAME 1
    end
    fsh \$FSH_LAST_STATUS
end" > $HOME/.config/fish/functions/fish_prompt.fish
    fi
fi
