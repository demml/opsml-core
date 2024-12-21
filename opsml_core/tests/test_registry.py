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

    cards = registry.list_cards(version="2.0.0")
    assert len(cards.cards) == 1

    cards = registry.list_cards(version="3.*")

    assert len(cards.cards) == 2
