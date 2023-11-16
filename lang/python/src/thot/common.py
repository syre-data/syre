import os

def dev_mode() -> bool:
    """
    Returns if the script is running in dev mode.
    
    Returns:
        bool: If the database is running in dev mode.
    """
    return os.getenv("THOT_CONTAINER_ID") is None