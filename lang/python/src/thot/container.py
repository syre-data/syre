from thot import _LEGACY_
from .types import OptStr, Tags, Metadata
from .asset import Asset

if _LEGACY_:
    from typing import List
    Assets = List[Asset]
else:
    Assets = list[Asset]

class Container:
    """
    A Container.
    """
    def __init__(
        self,
        rid: str,
        name: OptStr = None,
        type: OptStr = None,
        description: OptStr = None,
        tags: Tags = [],
        metadata: Metadata = {},
        assets: Assets = []
    ):
        """
        Create a new Container.
        """
        self._rid: str = rid
        self._name: OptStr = name
        self._type: OptStr = type
        self._description: OptStr = description
        self._tags: Tags = tags
        self._metadata: Metadata = metadata
        self._assets: Assets = assets
    
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
    def description(self) -> OptStr:
        """
        Returns:
            OptStr: Container's description.
        """
        return self._description
    
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
    def assets(self) -> Assets:
        """
        Returns:
            list[Asset]: Container's Assets.
        """
        return self._assets