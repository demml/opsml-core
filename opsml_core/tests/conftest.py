import pytest
from opsml_core import Feature, OpsmlMixin
from typing import Tuple, Dict
from pydantic import BaseModel


class MockInterface(BaseModel):
    is_interface: bool = True


@pytest.fixture
def card_args() -> Tuple[Dict[str, Feature], Dict[str, str]]:
    feature_map = {"feature1": Feature("type1", [1, 2, 3], {"arg1": "value1"})}
    metadata = {"key1": "value1"}
    return feature_map, metadata


@pytest.fixture
def mock_interface(
    card_args: Tuple[Dict[str, Feature], Dict[str, str]],
) -> MockInterface:
    feature_map, metadata = card_args
    return MockInterface()
