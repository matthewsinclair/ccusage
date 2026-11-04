# bin/.devbin/cmd/common/handler.zsh: what the project handlers beside it share. Each
# sources it straight after setting its options:
#
#   source ${0:A:h}/common/handler.zsh
#
# It names the handler, $handler, finds the project root, $root, and works from
# there. die and say prefix what they print with the handler's name, die on stderr
# with status 1, and usage prints the usage block of the handler's header: its
# "#   bin/devbin <handler>" line through the bare # after it. devbin's doctor passes
# over this directory, as over any in cmd/ not named <command>.d, so no command is
# made of it.
#
# THE ROOT IS THE HANDLER'S OWN PATH, NOT DEVBIN'S PROJECT_ROOT: the bats suites run a
# copy of a handler in a tree of its own, under a bin/devbin test whose PROJECT_ROOT
# names this checkout, and the copy must act on its own tree.

handler=${ZSH_ARGZERO:t}
root=${ZSH_ARGZERO:A:h:h:h:h}
cd $root

die() { print -u2 -r -- "$handler: $*"; exit 1 }
say() { print -r -- "$handler: $*" }
usage() { sed -n "/^#   bin\/devbin $handler/,/^#\$/p" ${ZSH_ARGZERO:A} | sed -e '$d' -e 's/^# \{0,1\}//' }
