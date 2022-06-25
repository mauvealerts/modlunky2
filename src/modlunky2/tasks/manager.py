# Idea:
# create
# take list of tasks at creation
# "start" function returns
#   memory stream - progress
#   anyio event - for complete
# let anyio spawn worker(s) on-demand?
#   no way to
# force everything to go through async thread
#   if threaded, dispatch to pool
#   if multiprocess, dispatch to worker
#
# separate, but closely related, concern: need portal back to the tkinter thread
# convenience: provide handling of completion?

from dataclasses import dataclass
from enum import IntEnum
from typing import Any, Generic, Iterable, TypeVar

from modlunky2.tasks.register import Task, TaskHandler


class TaskState(IntEnum):
    IN_PROGRESS = 1
    SUCCEEDED = 2
    FAILED = 3


T = TypeVar("T")


@dataclass(frozen=True)
class TaskStatus(Generic[T]):
    data: T
    state: TaskState
    progress: float


class TaskManager:
    def __init__(self, tasks: Iterable[Task[Any]]):
        pass


class TaskRun(Generic[T]):
    handler: TaskHandler[T]
