from opsml_core._opsml_core import RegistryTestHelper
from opsml_core import CardRegistry, RegistryType


def test_helper():
    helper = RegistryTestHelper()
    assert helper is not None
    helper.setup()

    registry = CardRegistry(RegistryType.Data)
    assert registry is not None

    cards = registry.list_cards()
    assert len(cards.cards) == 10

    cards.as_table()

    helper.cleanup()
