from typing import List, Union
import subprocess
import importlib.resources as pkg_resources
import socket
import os
from datetime import datetime
import platform
from uuid import uuid4 as uuid

import zmq

from .types import OptStr, Tags, Metadata
from .common import dict_to_container, dict_to_asset
from .asset import Asset
from .container import Container

OptTags = Union[Tags, None]
OptMetadata = Union[Metadata, None]

LOCALHOST = "127.0.0.1"
THOT_PORT = 7047

class Database:
    """
    A Thot Database.
    """
    def __init__(self, dev_root: OptStr = None):
        """
        Create a new Thot Database.
        """
        self._ctx: zmq.Context = zmq.Context()
        self._socket: zmq.Socket = self._ctx.socket(zmq.REQ)
        self._socket.setsockopt(zmq.SNDTIMEO, 10_000)
        self._socket.setsockopt(zmq.RCVTIMEO, 10_000)
        self._socket.connect(f'tcp://{LOCALHOST}:{THOT_PORT}')
        if not self._is_database_available():
            exe_base_name = "thot-local-database"
            bin_path = pkg_resources.files("thot").joinpath("bin")
            os_name = platform.system()
            if os_name == "Linux":
                exe_path = bin_path.joinpath(f"{exe_base_name}-x86_64-unknown-linux-gnu")
            elif os_name == "Darwin":
                mac_system = platform.processor()
                if mac_system == 'arm':
                    exe_path = bin_path.joinpath(f"{exe_base_name}-aarch64-apple-darwin")
                else:
                    exe_path = bin_path.joinpath(f"{exe_base_name}-x86_64-apple-darwin")
            elif os_name == "Windows":
                exe_path = bin_path.joinpath(f"{exe_base_name}-x86_64-pc-windows-msvc.exe")
            else:
                raise OSError()
            
            subprocess.Popen(exe_path, start_new_session=True)
        
        root_id: OptStr = os.getenv("THOT_CONTAINER_ID")
        if root_id is None:
            if dev_root is None:
                raise ValueError("`dev_root` must be set")
            
            self._root_path: str = dev_root
        else:
            self._socket.send_json({"ContainerCommand": {"GetPath": root_id}})
            root_path = self._socket.recv_json()
            self._root_path: str = root_path
            
        self._socket.send_json({"ProjectCommand": {"ResourceRootPath": self._root_path}})
        project_path = self._socket.recv_json()
        if "Ok" not in project_path:
            raise RuntimeError("could not get project path")
        
        project_path = project_path["Ok"]
        self._socket.send_json({"ProjectCommand": {"Load": project_path}})
        project = self._socket.recv_json()
        if "Ok" not in project:
            raise RuntimeError(f"could not load project")
        
        project = project["Ok"]
        self._socket.send_json({"GraphCommand": {"Load": project["rid"]}})
        graph = self._socket.recv_json()
        if "Ok" not in graph:
            raise RuntimeError("could not load graph")
        
        self._socket.send_json({"ContainerCommand": {"ByPath": self._root_path}})
        root = self._socket.recv_json()
        if root is None:
            raise RuntimeError("could not get root Container")
        
        self._root: str = root["rid"]
        
    def _is_database_available(self) -> bool:
        """
        Check if a database is already running.

        Returns:
            bool: `True` if a database is already running, `False` otherwise.
        """
        s = socket.socket()
        try:
            s.bind((LOCALHOST, THOT_PORT))
            
        except OSError as err:
            system = platform.system()
            if system == 'Darwin':
                if err.errno != 48:
                    raise err
            elif system == 'Linux':
                if err.errno != 98:
                    raise err
            elif system == 'Windows':
                if (not hasattr(err, "winerror")) or (err.winerror != 10048):
                    raise err
            else:
                raise err
            
        else:
            # socket not bound, no chance of database running
            return False
            
        self._socket.send_json({'DatabaseCommand': 'Id'})
        resp = self._socket.recv_json()
        return resp == "thot local database"
        
    @property
    def root(self) -> Container:
        """
        Returns the root Container.
        
        Returns:
            Container: Root Container.
        """
        self._socket.send_json({"ContainerCommand": {"GetWithMetadata": self._root}})
        root = self._socket.recv_json()
        if root is None:
            raise RuntimeError("Could not get root Container")

        if 'Err' in root:
            raise RuntimeError(f"Error getting root: {root['Err']}")
        
        root = dict_to_container(root)
        return root
                
    def find_containers(
        self,
        name: OptStr = None,
        type: OptStr = None,
        tags: OptTags = None,
        metadata: OptMetadata = None
    ) -> List[Container]:
        """
        Find Containers matching the filter.

        Args:
            name (OptStr, optional): Name filter. Defaults to None.
            type (OptStr, optional): Type filter. Defaults to None.
            tags (OptTags, optional): Tags filter. Defaults to None.
            metadata (OptMetadata, optional): Metadata filter. Defaults to None.

        Returns:
            List[Container]: Containers matching the filter.
        """
        f = {}
        if name is not None:
            f['name'] = name
        if type is not None:
            f['kind'] = type
        if tags is not None:
            f['tags'] = tags
        if metadata is not None:
            f['metadata'] = metadata
        
        self._socket.send_json({"ContainerCommand": {"FindWithMetadata": (self._root, f)}})
        containers = self._socket.recv_json()
        if 'Err' in containers:
            raise RuntimeError(f"Error getting containers: {root['Err']}")

        containers = map(dict_to_container, containers)
        return list(containers)
    
    def find_container(
        self,
        name: OptStr = None,
        type: OptStr = None,
        tags: OptTags = None,
        metadata: OptMetadata = None
    ) -> Union[Container, None]:
        """
        Find a single Container matching the filter.
        
        Args:
            name (OptStr, optional): Name filter. Defaults to None.
            type (OptStr, optional): Type filter. Defaults to None.
            tags (OptTags, optional): Tags filter. Defaults to None.
            metadata (OptMetadata, optional): Metadata filter. Defaults to None.
        
        Returns:
            Union[Container, None]: A Contianer, or `None`.
        """
        containers = self.find_containers(name = name, type = type, tags = tags, metadata = metadata)
        if len(containers) == 0:
            return None
        
        return containers[0]
    
    def find_assets(
        self,
        name: OptStr = None,
        type: OptStr = None,
        tags: OptTags = None,
        metadata: OptMetadata = None
    ) -> List[Asset]:
        """
        Find Assets matching the filter.
        
        Args:
            name (OptStr, optional): Name filter. Defaults to None.
            type (OptStr, optional): Type filter. Defaults to None.
            tags (OptTags, optional): Tags filter. Defaults to None.
            metadata (OptMetadata, optional): Metadata filter. Defaults to None.

        Returns:
            List[Asset]: Assets matching the filter.
        """
        f = {}
        if name is not None:
            f['name'] = name
        if type is not None:
            f['kind'] = type
        if tags is not None:
            f['tags'] = tags
        if metadata is not None:
            f['metadata'] = metadata
        
        self._socket.send_json({"AssetCommand": {"FindWithMetadata": (self._root, f)}})
        assets = self._socket.recv_json()
        if 'Err' in assets:
            raise RuntimeError(f"Error getting assets: {root['Err']}")

        assets = map(dict_to_asset, assets)
        return list(assets)
            
    def find_asset(
        self,
        name: OptStr = None,
        type: OptStr = None,
        tags: OptTags = None,
        metadata: OptMetadata = None    
    ) -> Union[Asset, None]:
        """
        Find a single Asset matching the filter.
        
        Args:
            name (OptStr, optional): Name filter. Defaults to None.
            type (OptStr, optional): Type filter. Defaults to None.
            tags (OptTags, optional): Tags filter. Defaults to None.
            metadata (OptMetadata, optional): Metadata filter. Defaults to None.
        
        Returns: 
            Union[Asset, None]: An Asset or `None`.
        """
        assets = self.find_assets(name = name, type = type, tags = tags, metadata = metadata)
        if len(assets) == 0:
            return None
        
        return assets[0]
        
    def add_asset(
        self,
        file: str,
        name: OptStr = None,
        type: OptStr = None,
        description: OptStr = None,
        tags: Tags = [],
        metadata: Metadata = {}    
    ) -> str:
        """
        Adds an Asset to the project.
        
        Args:
            file (str): File name of the associated data. Use relative paths to place the Asset in a bucket.
            name (OptStr, optional): Name filter. Defaults to None.
            type (OptStr, optional): Type filter. Defaults to None.
            tags (OptTags, optional): Tags filter. Defaults to None.
            metadata (OptMetadata, optional): Metadata filter. Defaults to None.

        Returns:
            str: Path to save the Asset's file to.
        """
        if os.path.isabs(file):
            raise ValueError("file must be relative")
        
        path = {"Relative": file}
        user = self._active_user()
        if user is None:
            raise RuntimeError("could not get active user")
        
        uid = user["rid"]
        properties = {
            "created": datetime.now().strftime("%Y-%m-%dT%H:%M:%SZ"),
            "creator": {"User": {"Id": uid}},
            "name": name,
            "kind": type,
            "description": description,
            "tags": tags,
            "metadata": metadata
        }
        
        asset = {
            'rid': str(uuid()),
            'properties': properties,
            'path': path
        }
        
        self._socket.send_json({"AssetCommand": {"Add": (asset, self._root)}})
        res = self._socket.recv_json()
        if "Ok" not in res:
            raise RuntimeError(f"could not create Asset: {res['Err']}")
        
        path = os.path.join(self._root_path, file)
        os.makedirs(os.path.dirname(path), exist_ok=True) # ensure bucket directory exists
        return path
    
    def _active_user(self) -> OptStr:
        """
        Get the active user.

        Returns:
            OptStr: Active user.
        """
        self._socket.send_json({"UserCommand": "GetActive"})
        user = self._socket.recv_json()
        if "Ok" not in user:
            return None
        
        return user["Ok"]
