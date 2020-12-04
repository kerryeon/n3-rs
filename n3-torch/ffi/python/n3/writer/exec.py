import glob
import os
import sys
import time

import inflection
import tensorboardX

from .epoch import EpochWriter
from ..util.args import Args
from ..util.dirs import LOGS_DIR


class ExecWriter:
    def __init__(self, args: Args, exec: str, model: str, root: bool) -> None:
        super().__init__()
        self._exec_name = inflection.underscore(exec)
        self._model_name = inflection.underscore(model)

        if root:
            logdir = os.path.join(args['env']['root'], LOGS_DIR,
                                  self._exec_name, self._model_name)
            logdir = _increment_dir(logdir)
            self._writer = tensorboardX.SummaryWriter(logdir)
        else:
            self._writer = None

        self._epoch_writer = None

        self._epoch = args['epoch']
        self._rust_kwargs = None

    def attach_rust_kwargs(self, kwargs):
        self._rust_kwargs = kwargs

    def is_running(self) -> bool:
        return self._rust_kwargs['is_running']

    def do_epoch(self, tag, fn_dataset):
        tag = f'{self._exec_name}/{tag}'
        self._epoch_writer = EpochWriter(
            self._writer, tag, fn_dataset, 0, self._epoch)
        return self._epoch_writer

    def update_rust_kwargs(self, metrics):
        # should be called on root
        assert self._rust_kwargs is not None
        # update time
        time_total_secs = self._epoch_writer.time_total_secs
        if time_total_secs:
            time_secs, _ = self._rust_kwargs['date_begin']
            time_secs += time_total_secs
            self._rust_kwargs['date_end'] = (time_secs, 0)

    def close(self):
        self._epoch_writer = None

        if self._writer is not None:
            self._writer.close()
            self._writer = None
        if self._rust_kwargs is not None:
            # update rust writer
            self._rust_kwargs['is_running'] = False
            ts = time.time_ns()
            ts_secs = ts // 1_000_000_000
            ts_nsecs = ts % 1_000_000_000
            self._rust_kwargs['date_end'] = (ts_secs, ts_nsecs)
            self._rust_kwargs = None


def spawn_daemon(env):
    from tensorboard.main import run_main

    logdir = os.path.join(env.root, LOGS_DIR)
    sys.argv = [sys.argv[0]] + ['--logdir', logdir] + sys.argv[3:]
    run_main()


def _increment_dir(logdir):
    num_dirs = len(glob.glob(rf'{logdir}/exp*/'))
    return os.path.join(logdir, f'exp{num_dirs}')
