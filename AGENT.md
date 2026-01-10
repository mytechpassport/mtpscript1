# AI Agent Rules for MTPScript Development

## File Organization Rules
- **Always consult `requirements/FOLDER_STRUCTURE.md`** before placing any files in the project
- **Never randomly place files** - follow the established directory structure and file placement guidelines
- **Document new directories** in `FOLDER_STRUCTURE.md` when creating them
- **Follow existing patterns** for similar functionality

## Testing Methodology
- **Golden test method is key** for our language development and validation
- **Create fixture program tests first** before implementing any feature
- **Fixture tests serve as acceptance tests** that validate feature implementation
- **Tests must not be changed after creation** to ensure we don't make tests easier just to pass them
- **Fixture tests should be runnable** and provide clear validation of expected behavior

## Development Workflow
0. **When something is not clear, consult `requirements/TECHSPECV5.md`** for authoritative specifications
1. **Read requirements** and understand the feature specification
2. **Create fixture test** that defines expected behavior
3. **Implement feature** to satisfy the test
4. **Run test** to validate implementation
5. **Document changes** in appropriate files following folder structure rules
