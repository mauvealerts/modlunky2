from abc import ABC, abstractmethod
from dataclasses import dataclass
from enum import IntEnum
from typing import Callable, Generic, Type, TypeVar


class TaskStrategy(IntEnum):
    ASYNC = 1
    THREAD = 2
    PROCESS = 3


T = TypeVar("T")


class Task(ABC, Generic[T]):
    data: T

    @abstractmethod
    def update_progress(self, ratio: float):
        pass


TaskHandler = Callable[[Task[T]], None]


@dataclass(frozen=True)
class TaskSpec(Generic[T]):
    data_type: Type[T]
    handler: TaskHandler[T]
    strategy: TaskStrategy
