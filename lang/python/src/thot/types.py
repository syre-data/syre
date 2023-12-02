from typing import Union, Any
from thot import _LEGACY_
    
OptStr = Union[str, None]

if _LEGACY_:
    from typing import List, Dict
    Tags = List[str]
    Metadata = Dict[str, Any]
    Properties = Dict[str, Any]
else:
    Tags = list[str]
    Metadata = dict[str, Any]
    Properties = dict[str, Any]