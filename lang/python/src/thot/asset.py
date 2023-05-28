from .types import OptStr, Tags, Metadata

class Asset:
    """
    An Asset.
    """
    def __init__(
        self,
        rid: str,
        file: str,
        name: OptStr = None,
        type: OptStr = None,
        tags: Tags = [],
        metadata: Metadata = {},
    ):
        """
        Create a new Asset.
        """
        self._rid: str = rid
        self._file: str = file
        self._name: OptStr = name
        self._type: OptStr = type
        self._tags: Tags = tags
        self._metadata: Metadata = metadata
    
    @property
    def name(self) -> OptStr:
        """
        Returns:
            OptStr: Asset's name.
        """
        return self._name
    
    @property
    def type(self) -> OptStr:
        """
        Returns:
            OptStr: Asset's type.
        """
        return self._type
    
    @property
    def tags(self) -> Tags:
        """
        Returns:
            Tags: Asset's tags.
        """
        return self._tags
    
    @property
    def metadata(self) -> Metadata:
        """
        Returns:
            Metadata: Asset's metadata.
        """
        return self._metadata
        
    @property
    def file(self) -> str:
        """
        Returns:
            str: Asset's file path.
        """
        return self._file