import builtins
import typing
from typing import Any, Iterable, Union
from functools import partial

from .resources import Container, Asset

Resource = Union[Asset, Container]

def property_filter(prop: str, value: Any, obj: Resource):
    """
    Check if object matched property filter.

    Args:
        prop (str): Name of the property to check.
            Properties are drilled down using dot notation.
            
        value (Any): Value to match.
            Value can be a
            + List: Object should be a list.
                Returns True if all values in object are included in value.
            + Dictionary: Uses operators to match.
                Operators are
                + $in: Tests for inclusion.
                    + If object to check is not a list,
                        checks if the object is in the list. 
                    + If object to check is a list,
                        checks if all elements are in the value list.
            + Callable: Value is passed the object,
                and should return a boolean indicating if the object matches.
            + RegEx: RegEx#search is used to match object.
            + String: Tests if value and object are equal. 
            
        obj (Resource): Object to check.
        
    Returns:
        True if matches, False otherwise.
    """
    # parse prop
    prop_path = prop.split('.')
    for part in prop_path:
        if isinstance(obj, dict):
            try:
                obj = obj[part]
                
            except KeyError as err:
                # property not contained in object
                return False
        
        else:
            try:
                obj = getattr(obj, part)

            except AttributeError as err:
                return False

    if isinstance(value, list):
        # value is list, check for inclusion
        if isinstance(obj, list):
            # object is list, verfiy all values are in object
            for item in value:
                if isinstance(item, typing.Pattern):
                    # value is regex
                    match = item.search(value)
                    return (match is None)

                if item not in obj:
                    # search item not obj
                    return False

            # all items present
            return True


        else:
            # object is not list, can not match
            return False

    elif isinstance(value, dict):
        # value is dictionary, search for operators
        for op, val in value.items():
            if op == '$in':
                # inclusion operator
                if not isinstance(val, list):
                    raise TypeError(f'Invalid search criteria {op}: {val}. Value must be list.')

                if isinstance(obj, list):
                    # test all values are included in object
                    return all([(v in obj) for v in val])

                else:
                    # basic object
                    return (obj in val)

            elif op == '$eq':
                # equals operator
                return (obj == val)

            else:
                # not an operator
                raise TypeError(f'Invalid search operator {op}')

        # passed all operator checks
        return True

    elif callable(value):
        # value is a function
        return value(obj)

    else:
        # value is not list, check for direct match
        if isinstance(value, typing.Pattern):
            # value is regex
            match = value.search(obj)
            return (match is not None)

        # simple value
        return (obj == value)


def filter(filter: Any, resources: Iterable[Resource]):
    """
    Filters an iterable of Assets and/or Containers.

    Args:
        filter: Search criteria.
            [See #property_filter > `value` for more info]
        Resources (Iterable[Resource]): Iterable of resources to filter.
    
    Returns:
        List of filtered resources.
    """
    matching = resources.copy()
    for prop, value in filter.items():
        obj_fltr = partial( property_filter, prop, value )
        matching = builtins.filter( obj_fltr, matching )  # builtins required because filter overwritten by parameter.

    return list( matching )
