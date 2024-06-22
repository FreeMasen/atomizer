#! /bin/bash


function write_line() {
    echo "$1" >> README.md
}

function open_code_fence() {
    write_line "\`\`\`console"
}

function close_code_fence() {
    write_line "\`\`\`"
}

function cargo_run_help() {
    if [ -z "$1" ]
    then
        write_line "atomizer --help"
        cargo run -- --help >> README.md
    else
        write_line "atomizer $1 --help"
        cargo run -- "$1" --help >> README.md
    fi

}

function do_sub_command() {
    write_line
    write_line "### \`$1\`"
    write_line
    open_code_fence
    cargo_run_help "$1"
    close_code_fence
}

echo "# Atomizer" > README.md
write_line
write_line "A terminal Atom feed reader"
write_line

write_line "## Usage"
write_line

open_code_fence
cargo_run_help 
close_code_fense

do_sub_command "read"
do_sub_command "categories"
do_sub_command "update"
do_sub_command "setup"
do_sub_command "add"
do_sub_command "remove"
do_sub_command "config"
