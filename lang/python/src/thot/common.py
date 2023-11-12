import os
from typing import Any

from thot import _LEGACY_
from .container import Container
from .asset import Asset

if _LEGACY_:
    from typing import Dict
    Properties = Dict[str, Any]
else:
    Properties = dict[str, Any]


def dev_mode() -> bool:
    """
    Returns if the script is running in dev mode.
    
    Returns:
        bool: If the database is running in dev mode.
    """
    return os.getenv("THOT_CONTAINER_ID") is None

def dict_to_asset(d: Properties) -> Asset:
    """
    Converts a dictionary to an Asset.

    Args:
        d (dict[str, Any]): Dictionary to convert.

    Returns:
        Asset: Asset that the JSON represented.
    """
    file = d["path"]
    if "Absolute" not in file:
        raise ValueError("Asset path must be absolute")
    
    return Asset(
        d["rid"],
        file["Absolute"],
        name = d["properties"]["name"],
        type = d["properties"]["kind"],
        description = d["properties"]["description"],
        tags = d["properties"]["tags"],
        metadata = d["properties"]["metadata"]
    )

def dict_to_container(d: Properties) -> Container:
    """
    Converts a dictionary to a Container.

    Args:
        d (dict[str, Any]): Dictionary to convert.

    Returns:
        Container: Container that the JSON represented.
    """
    assets = list(map(lambda asset: dict_to_asset(asset), d["assets"].values()))

    return Container(
        d["rid"],
        name = d["properties"]["name"],
        type = d["properties"]["kind"],
        description = d["properties"]["description"],
        tags = d["properties"]["tags"],
        metadata = d["properties"]["metadata"],
        assets = assets
    )
    
