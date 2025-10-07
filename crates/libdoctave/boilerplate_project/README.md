
# Docapella Starter Template

This is a starter template to help you get familiar with how Docapella works.

## Where to start?

Open this file (`README.md`) in your editor of choice and make a change to it. You will see Docapella Studio update immediately when you save your changes.

You can also look at how you can use Docapella's component system to create complex layouts like this:

<Grid cols="2" gap="3">
  <Card>
    <Flex justify="between" gap="2" height="full" dir="column">
        <Box>
            <Icon set="lucide" name="server" variant="boxed" color size="md" />

            #### OpenAPI specifications

            Add your own OpenAPI specification to this project and generate your own API documentation.
        </Box>

        <Button href="/api">Learn More</Button>
    </Flex>
  </Card>

  <Card>
    <Flex justify="between" gap="2" height="full" dir="column">
        <Box>
            <Icon set="lucide" name="boxes" variant="boxed" color size="md" />

            #### Components

            Add engaging UI and layout components to your documentation to make them more engaging.
        </Box>

        <Button href="/components">Learn More</Button>
    </Flex>
  </Card>

  <Card>
    <Flex justify="between" gap="2" height="full" dir="column">
        <Box>
            <Icon set="lucide" name="table-2" variant="boxed" color size="md" />

            #### Tabs and Subtabs

            Tabs and subtabs are used to structure your documentations site into logical groups.
        </Box>

        <Button href="/tabs.md">Learn More</Button>
    </Flex>
  </Card>
</Grid>

