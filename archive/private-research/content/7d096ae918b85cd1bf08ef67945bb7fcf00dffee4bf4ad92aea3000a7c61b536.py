"""Fixed run-002 context. Defaults to verification; never grants approval.

Actions: prepare [--stage], launch (dry run by default), gate (read only by
default), archive (unreleased attempt only), finish (exited process only).
Original launch/release requires a separately recorded human approval.
"""
import session


def main():
    session.main('run-002')


if __name__=='__main__': main()
