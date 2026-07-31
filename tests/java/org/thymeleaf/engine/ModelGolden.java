package org.thymeleaf.engine;

import java.util.Collections;
import java.util.ArrayList;
import java.util.List;
import java.io.StringWriter;
import java.util.LinkedHashSet;

import org.thymeleaf.IEngineConfiguration;
import org.thymeleaf.context.TestTemplateEngineConfigurationBuilder;
import org.thymeleaf.model.AbstractModelVisitor;
import org.thymeleaf.model.IModel;
import org.thymeleaf.model.IText;
import org.thymeleaf.templatemode.TemplateMode;
import org.thymeleaf.templateresource.StringTemplateResource;
import org.thymeleaf.cache.AlwaysValidCacheEntryValidity;

/** 固定可变 Model 的编辑、事件身份和配置兼容性语义。 */
public final class ModelGolden {
    private ModelGolden() { }

    public static void main(final String[] args) throws Exception {
        final IEngineConfiguration configuration =
                TestTemplateEngineConfigurationBuilder.build(Collections.emptySet());
        final TemplateModel templateModel = new TemplateModel(
                configuration,
                new TemplateData("template", null, null, TemplateMode.HTML, null),
                new IEngineTemplateEvent[] {
                    TemplateStart.TEMPLATE_START_INSTANCE, TemplateEnd.TEMPLATE_END_INSTANCE
                });
        final TemplateData templateData = new TemplateData(
                "template", null, null, TemplateMode.HTML, null);
        System.out.println("templateData=" + templateData.getTemplate() + ","
                + templateData.hasTemplateSelectors() + "," + templateData.getTemplateSelectors() + ","
                + templateData.getTemplateResource() + "," + templateData.getTemplateMode() + ","
                + templateData.getValidity());
        final StringTemplateResource resource = new StringTemplateResource("contents");
        final AlwaysValidCacheEntryValidity validity = new AlwaysValidCacheEntryValidity();
        final TemplateData fullTemplateData = new TemplateData(
                "full", new LinkedHashSet<>(List.of("second", "first")), resource,
                TemplateMode.XML, validity);
        System.out.println("templateDataFull=" + fullTemplateData.getTemplate() + ","
                + fullTemplateData.getTemplateSelectors() + ","
                + (fullTemplateData.getTemplateResource() == resource) + ","
                + fullTemplateData.getTemplateResource().getDescription() + ","
                + fullTemplateData.getTemplateMode() + ","
                + (fullTemplateData.getValidity() == validity));
        final String immutableMessage = failureMessage(() -> templateModel.add(null));
        final boolean immutableOperations = immutableMessage.equals(failureMessage(() -> templateModel.insert(0, null)))
                && immutableMessage.equals(failureMessage(() -> templateModel.replace(0, null)))
                && immutableMessage.equals(failureMessage(() -> templateModel.addModel(null)))
                && immutableMessage.equals(failureMessage(() -> templateModel.insertModel(0, null)))
                && immutableMessage.equals(failureMessage(() -> templateModel.remove(0)))
                && immutableMessage.equals(failureMessage(templateModel::reset));
        final IModel mutableTemplateModel = templateModel.cloneModel();
        System.out.println("templateModel=" + templateModel.size() + "," + templateModel.getTemplateMode()
                + "," + immutableOperations + "," + mutableTemplateModel.size());
        System.out.println("immutable=" + immutableMessage);
        final RecordingHandler boundaryHandler = new RecordingHandler();
        TemplateStart.TEMPLATE_START_INSTANCE.beHandled(boundaryHandler);
        TemplateEnd.TEMPLATE_END_INSTANCE.beHandled(boundaryHandler);
        final RecordingVisitor boundaryVisitor = new RecordingVisitor();
        TemplateStart.TEMPLATE_START_INSTANCE.accept(boundaryVisitor);
        TemplateEnd.TEMPLATE_END_INSTANCE.accept(boundaryVisitor);
        final StringWriter boundariesWriter = new StringWriter();
        TemplateStart.TEMPLATE_START_INSTANCE.write(boundariesWriter);
        TemplateEnd.TEMPLATE_END_INSTANCE.write(boundariesWriter);
        System.out.println("boundaries="
                + (TemplateStart.TEMPLATE_START_INSTANCE == TemplateStart.asEngineTemplateStart(null)) + ","
                + (TemplateEnd.TEMPLATE_END_INSTANCE == TemplateEnd.asEngineTemplateEnd(null)) + ","
                + boundariesWriter + "," + boundaryHandler.starts + "," + boundaryHandler.ends + ","
                + boundaryVisitor.starts + "," + boundaryVisitor.ends);
        final Model model = new Model(configuration, TemplateMode.HTML);
        final Text a = new Text("a");
        final Text b = new Text("b");
        final Text c = new Text("c");
        model.add(a);
        model.insert(0, b);
        model.replace(1, c);
        System.out.println("edited=" + model.size() + "," + model + "," + (model.get(0) == b)
                + "," + (model.get(1) == c));
        model.insert(3, null);
        System.out.println("nullInsert=" + model.size() + "," + model);
        final IModel clone = model.cloneModel();
        System.out.println("clone=" + (clone.get(0) == model.get(0)) + "," + clone);
        final Model same = new Model(configuration, TemplateMode.HTML);
        same.addModel(model);
        System.out.println("sameConfig=" + same);
        final RecordingHandler handler = new RecordingHandler();
        model.process(handler);
        System.out.println("dispatch=" + handler.text());
        handler.clear();
        final TemplateFlowController controller = new TemplateFlowController();
        System.out.println("throttled=" + model.process(handler, 1, controller) + "," + handler.text());
        controller.stopProcessing = true;
        System.out.println("stopped=" + model.process(handler, 0, controller));
        final RecordingVisitor visitor = new RecordingVisitor();
        model.accept(visitor);
        System.out.println("visitor=" + visitor.text());
        final Model resetClone = new Model(configuration, TemplateMode.XML);
        resetClone.add(new Text("different"));
        resetClone.resetAsCloneOf(model);
        System.out.println("resetClone=" + resetClone.getTemplateMode() + "," + resetClone
                + "," + resetClone.sameAs(model));
        resetClone.replace(0, new Text("x"));
        System.out.println("sameAsChanged=" + resetClone.sameAs(model));

        final IEngineConfiguration otherConfiguration =
                TestTemplateEngineConfigurationBuilder.build(Collections.emptySet());
        final Model other = new Model(otherConfiguration, TemplateMode.HTML);
        other.add(new Text("other"));
        try {
            model.addModel(other);
            System.out.println("differentConfig=NONE");
        } catch (final RuntimeException exception) {
            System.out.println("differentConfig=" + exception.getClass().getSimpleName() + ":"
                    + exception.getMessage());
        }
        final Model differentMode = new Model(configuration, TemplateMode.XML);
        differentMode.add(new Text("xml"));
        try {
            model.addModel(differentMode);
            System.out.println("differentMode=NONE");
        } catch (final RuntimeException exception) {
            System.out.println("differentMode=" + exception.getClass().getSimpleName() + ":"
                    + exception.getMessage());
        }
        model.reset();
        System.out.println("reset=" + model.size() + "," + model);
    }

    private static final class RecordingHandler extends AbstractTemplateHandler {
        private final List<String> texts = new ArrayList<>();
        private int starts;
        private int ends;

        @Override
        public void handleTemplateStart(final org.thymeleaf.model.ITemplateStart templateStart) {
            this.starts++;
        }

        @Override
        public void handleTemplateEnd(final org.thymeleaf.model.ITemplateEnd templateEnd) {
            this.ends++;
        }

        @Override
        public void handleText(final IText text) {
            this.texts.add(text.getText().toString());
        }

        String text() {
            return String.join("", this.texts);
        }

        void clear() {
            this.texts.clear();
        }
    }

    private static final class RecordingVisitor extends AbstractModelVisitor {
        private final List<String> texts = new ArrayList<>();
        private int starts;
        private int ends;

        @Override
        public void visit(final org.thymeleaf.model.ITemplateStart templateStart) {
            this.starts++;
        }

        @Override
        public void visit(final org.thymeleaf.model.ITemplateEnd templateEnd) {
            this.ends++;
        }

        @Override
        public void visit(final IText text) {
            this.texts.add(text.getText().toString());
        }

        String text() {
            return String.join("", this.texts);
        }
    }

    private static String failureMessage(final Runnable operation) {
        try {
            operation.run();
            return "NONE";
        } catch (final RuntimeException exception) {
            return exception.getClass().getSimpleName() + ":" + exception.getMessage();
        }
    }
}
