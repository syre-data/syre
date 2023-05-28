from typing import List

from .types import OptStr, Tags, Metadata
from .asset import Asset

class Container:
    """
    A Container.
    """
    def __init__(
        self,
        rid: str,
        name: OptStr = None,
        type: OptStr = None,
        tags: Tags = [],
        metadata: Metadata = {},
        assets: List[str] = []
    ):
        """
        Create a new Container.
        """
        self._rid: str = rid
        self._name: OptStr = name
        self._type: OptStr = type
        self._tags: Tags = tags
        self._metadata: Metadata = metadata
        self._assets: List[str] = assets
    
    @property
    def name(self) -> OptStr:
        """
        Returns:
            OptStr: Container's name.
        """
        return self._name
    
    @property
    def type(self) -> OptStr:
        """
        Returns:
            OptStr: Container's type.
        """
        return self._type
    
    @property
    def tags(self) -> Tags:
        """
        Returns:
            Tags: Container's tags.
        """
        return self._tags
    
    @property
    def metadata(self) -> Metadata:
        """
        Returns:
            Metadata: Container's metadata.
        """
        return self._metadata
        
    @property
    def assets(self) -> List[Asset]:
        """
        Returns:
            List[Asset]: Container's Assets.
        """
        return self._assets