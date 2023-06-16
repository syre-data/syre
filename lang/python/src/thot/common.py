import os
from typing import Any

from .container import Container
from .asset import Asset

def dev_mode() -> bool:
    """
    Returns if the script is running in dev mode.
    
    Returns:
        bool: If the database is running in dev mode.
    """
    return os.getenv("THOT_CONTAINER_ID") is None

def dict_to_container(d: dict[str, Any]) -> Container:
    """
    Converts a dictionary to a Container.

    Args:
        d (dict[str, Any]): Dictionary to convert.

    Returns:
        Container: Container that the JSON represented.
    """
    return Container(
        d["rid"],
        name = d["properties"]["name"],
        type = d["properties"]["kind"],
        tags = d["properties"]["tags"],
        metadata = d["properties"]["metadata"],
        assets = d["assets"]
    )
    
def dict_to_asset(d: dict[str, Any]) -> Asset:
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
        tags = d["properties"]["tags"],
        metadata = d["properties"]["metadata"]
    )