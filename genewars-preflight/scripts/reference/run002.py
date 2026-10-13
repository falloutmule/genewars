"""Fixed run-002 context. Defaults to verification; never grants approval.

Actions: prepare [--stage], launch (dry run by default), gate (read only by
default), archive (unreleased attempt only), finish (exited process only).
Original launch/release requires a separately recorded human approval.
"""
import importlib
import sys
import prepare


def main():
    action = sys.argv[1] if len(sys.argv)>1 else 'prepare'
    modules = {'prepare':'prepare','launch':'launch','gate':'gate','archive':'archive_failed_launch','finish':'finish002'}
    if action not in modules:
        raise SystemExit('usage: run002.py {prepare|launch|gate|archive|finish} [reviewed action flags]')
    prepare.select_session('run-002')
    if len(sys.argv)>1:
        del sys.argv[1]
    importlib.import_module(modules[action]).main()


if __name__=='__main__': main()
