from opsml_core._opsml_core import RegistryTestHelper
from opsml_core import CardRegistry, RegistryType


def test_registry(mock_db):
    registry = CardRegistry(RegistryType.Data)
    assert registry is not None

    cards = registry.list_cards()
    assert len(cards.cards) == 10

    cards.as_table()


def test_registry_version(mock_db):
    registry = CardRegistry(RegistryType.Data)
    assert registry is not None

    cards = registry.list_cards()
    assert len(cards.cards) == 10

    cards.as_table()
