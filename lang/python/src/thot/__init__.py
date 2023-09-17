# SPDX-FileCopyrightText: 2023-present Brian Carlsen <carlsen.bri@gmail.com>
#
# SPDX-License-Identifier: MIT
import sys
if sys.version_info < (3, 7):
    raise RuntimeError("Requires Python version 3.7 or higher")

if sys.version_info < (3, 9):
    _LEGACY_ = True
else:
    _LEGACY_ = False

from .common import dev_mode
from .database import Database
from .filter import filter