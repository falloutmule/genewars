"""Fixed reviewed reference-session routing; no action grants human permission."""
import importlib
import sys
import prepare

def main(session):
    modules={'prepare':'prepare','launch':'launch','gate':'gate','archive':'archive_failed_launch','finish':'finish002'}
    action=sys.argv[1] if len(sys.argv)>1 else 'prepare'
    if action not in modules:
        raise SystemExit('usage: '+prepare.SESSION_WRAPPERS[session]+' {prepare|launch|gate|archive|finish} [reviewed action flags]')
    prepare.select_session(session)
    if len(sys.argv)>1:del sys.argv[1]
    importlib.import_module(modules[action]).main()
